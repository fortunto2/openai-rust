// Automatic cursor-based pagination for list endpoints.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::error::OpenAIError;

/// A page of results from a list endpoint.
pub struct Page<T> {
    /// The items on this page.
    pub data: Vec<T>,
    /// Whether there are more pages after this one.
    pub has_more: bool,
    /// Cursor for the next page (typically `last_id` or last item's `id`).
    pub after_cursor: Option<String>,
}

/// Type alias for the boxed future returned by the page fetcher.
#[cfg(not(target_arch = "wasm32"))]
type PageFuture<T> = Pin<Box<dyn Future<Output = Result<Page<T>, OpenAIError>> + Send>>;
#[cfg(target_arch = "wasm32")]
type PageFuture<T> = Pin<Box<dyn Future<Output = Result<Page<T>, OpenAIError>>>>;

/// Type alias for the page fetcher closure.
#[cfg(not(target_arch = "wasm32"))]
type PageFetcher<T> = Box<dyn Fn(Option<String>) -> PageFuture<T> + Send + Sync>;
#[cfg(target_arch = "wasm32")]
type PageFetcher<T> = Box<dyn Fn(Option<String>) -> PageFuture<T>>;

/// An async stream that automatically paginates through all results.
///
/// Created by `list_auto()` methods on resources. Yields individual items
/// across pages, fetching the next page only when all items from the current
/// page have been consumed.
///
/// # Example
///
/// ```ignore
/// use futures_util::StreamExt;
///
/// let mut stream = client.files().list_auto(FileListParams::new());
/// while let Some(file) = stream.next().await {
///     let file = file?;
///     println!("{}: {}", file.id, file.filename);
/// }
/// ```
pub struct Paginator<T> {
    /// Closure that fetches a page given an optional `after` cursor.
    fetch: PageFetcher<T>,
    /// Items remaining on the current page (reversed for pop efficiency).
    buffer: Vec<T>,
    /// Cursor for the next page.
    next_cursor: Option<String>,
    /// Whether we've exhausted all pages.
    done: bool,
    /// In-flight page fetch future.
    pending: Option<PageFuture<T>>,
}

impl<T> Paginator<T> {
    /// Create a new paginator with a page-fetching closure.
    ///
    /// The closure receives `Option<String>` (`None` for first page, `Some(cursor)` for
    /// subsequent pages) and returns a future resolving to `Result<Page<T>, OpenAIError>`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<F, Fut>(fetch: F) -> Self
    where
        F: Fn(Option<String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Page<T>, OpenAIError>> + Send + 'static,
    {
        Self {
            fetch: Box::new(move |cursor| Box::pin(fetch(cursor))),
            buffer: Vec::new(),
            next_cursor: None,
            done: false,
            pending: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new<F, Fut>(fetch: F) -> Self
    where
        F: Fn(Option<String>) -> Fut + 'static,
        Fut: Future<Output = Result<Page<T>, OpenAIError>> + 'static,
    {
        Self {
            fetch: Box::new(move |cursor| Box::pin(fetch(cursor))),
            buffer: Vec::new(),
            next_cursor: None,
            done: false,
            pending: None,
        }
    }
}

impl<T: Unpin + Send + 'static> Stream for Paginator<T> {
    type Item = Result<T, OpenAIError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // Drain buffered items first.
        if let Some(item) = this.buffer.pop() {
            return Poll::Ready(Some(Ok(item)));
        }

        // All pages consumed.
        if this.done {
            return Poll::Ready(None);
        }

        // Start a new fetch if none is in flight.
        if this.pending.is_none() {
            let cursor = this.next_cursor.take();
            this.pending = Some((this.fetch)(cursor));
        }

        // Poll the in-flight fetch.
        let fut = this.pending.as_mut().unwrap();
        match Pin::new(fut).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => {
                this.pending = None;
                this.done = true;
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(Ok(page)) => {
                this.pending = None;
                if !page.has_more {
                    this.done = true;
                }
                this.next_cursor = page.after_cursor;

                // Reverse so we can pop from the end efficiently.
                let mut items = page.data;
                items.reverse();
                this.buffer = items;

                // Yield the first item (if any).
                match this.buffer.pop() {
                    Some(item) => Poll::Ready(Some(Ok(item))),
                    None => {
                        this.done = true;
                        Poll::Ready(None)
                    }
                }
            }
        }
    }
}

// Paginator is Unpin because all state is owned / boxed.
impl<T> Unpin for Paginator<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_single_page() {
        let paginator = Paginator::new(|cursor| async move {
            assert!(cursor.is_none(), "single page should not request next");
            Ok(Page {
                data: vec![1, 2, 3],
                has_more: false,
                after_cursor: None,
            })
        });

        let items: Vec<i32> = paginator
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_multi_page() {
        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let paginator = Paginator::new(move |cursor| {
            let cc = cc.clone();
            async move {
                let call = cc.fetch_add(1, Ordering::SeqCst);
                match call {
                    0 => {
                        assert!(cursor.is_none());
                        Ok(Page {
                            data: vec![1, 2],
                            has_more: true,
                            after_cursor: Some("cursor_1".into()),
                        })
                    }
                    1 => {
                        assert_eq!(cursor.as_deref(), Some("cursor_1"));
                        Ok(Page {
                            data: vec![3, 4],
                            has_more: true,
                            after_cursor: Some("cursor_2".into()),
                        })
                    }
                    2 => {
                        assert_eq!(cursor.as_deref(), Some("cursor_2"));
                        Ok(Page {
                            data: vec![5],
                            has_more: false,
                            after_cursor: None,
                        })
                    }
                    _ => panic!("unexpected call"),
                }
            }
        });

        let items: Vec<i32> = paginator
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_empty_page() {
        let paginator: Paginator<i32> = Paginator::new(|_| async {
            Ok(Page {
                data: vec![],
                has_more: false,
                after_cursor: None,
            })
        });

        let items: Vec<Result<i32, OpenAIError>> = paginator.collect().await;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_error_propagation() {
        let paginator: Paginator<i32> =
            Paginator::new(|_| async { Err(OpenAIError::InvalidArgument("test error".into())) });

        let results: Vec<Result<i32, OpenAIError>> = paginator.collect().await;
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }
}
