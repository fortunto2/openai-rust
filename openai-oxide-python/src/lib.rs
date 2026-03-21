//! Python bindings for openai-oxide via PyO3.
//!
//! Install: `cd openai-oxide-python && uv sync && uv run maturin develop`
//! Usage:   `from openai_oxide import Client`

mod stream;

use ::openai_oxide::config::ClientConfig;
use ::openai_oxide::types::responses::*;
use ::openai_oxide::OpenAI;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use crate::stream::PyResponseStream;

/// Convert OpenAI errors to Python RuntimeError.
fn to_py_err(e: ::openai_oxide::OpenAIError) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

/// Fastest OpenAI client — Rust core, Python interface.
///
/// ```python
/// from openai_oxide import Client
///
/// client = Client()  # uses OPENAI_API_KEY env
/// response = client.create("gpt-5.4", "Hello!")
/// print(response["text"])
/// ```
#[pyclass]
struct Client {
    inner: OpenAI,
}

#[pymethods]
impl Client {
    /// Create client from API key. If None, reads OPENAI_API_KEY env var.
    #[new]
    #[pyo3(signature = (api_key=None, base_url=None))]
    fn new(api_key: Option<String>, base_url: Option<String>) -> PyResult<Self> {
        let key = match api_key {
            Some(k) => k,
            None => std::env::var("OPENAI_API_KEY")
                .map_err(|_| PyRuntimeError::new_err("OPENAI_API_KEY not set"))?,
        };
        let mut config = ClientConfig::new(&key);
        if let Some(url) = base_url {
            config = config.base_url(url);
        }
        Ok(Self {
            inner: OpenAI::with_config(config),
        })
    }

    /// Create a response (Responses API).
    ///
    /// Returns dict with "id", "text", "model", "usage".
    ///
    /// ```python
    /// r = await client.create("gpt-5.4", "What is 2+2?")
    /// print(r["text"])  # "4"
    /// ```
    #[pyo3(signature = (model, input, max_output_tokens=None, instructions=None, temperature=None))]
    fn create<'py>(
        &self,
        py: Python<'py>,
        model: String,
        input: String,
        max_output_tokens: Option<i64>,
        instructions: Option<String>,
        temperature: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let mut req = ResponseCreateRequest::new(&model).input(input.as_str());
            if let Some(max) = max_output_tokens {
                req = req.max_output_tokens(max);
            }
            if let Some(inst) = instructions {
                req = req.instructions(inst);
            }
            if let Some(temp) = temperature {
                req = req.temperature(temp);
            }

            let resp = client.responses().create(req).await.map_err(to_py_err)?;
            response_to_dict(&resp)
        })
    }

    /// Create with structured output (JSON Schema).
    ///
    /// ```python
    /// r = await client.create_structured("gpt-5.4", "List 3 planets", "planets", schema)
    /// data = json.loads(r["text"])
    /// ```
    #[pyo3(signature = (model, input, schema_name, schema, max_output_tokens=None))]
    fn create_structured<'py>(
        &self,
        py: Python<'py>,
        model: String,
        input: String,
        schema_name: String,
        schema: String,
        max_output_tokens: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let schema_val: serde_json::Value =
                serde_json::from_str(&schema).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

            let mut req = ResponseCreateRequest::new(&model)
                .input(input.as_str())
                .text(ResponseTextConfig {
                    format: Some(ResponseTextFormat::JsonSchema {
                        name: schema_name,
                        description: None,
                        schema: Some(schema_val),
                        strict: Some(true),
                    }),
                    verbosity: None,
                });
            if let Some(max) = max_output_tokens {
                req = req.max_output_tokens(max);
            }

            let resp = client.responses().create(req).await.map_err(to_py_err)?;
            response_to_dict(&resp)
        })
    }

    /// Create with function calling tools.
    ///
    /// Returns dict with "text", "function_calls" (list of {name, arguments, call_id}).
    #[pyo3(signature = (model, input, tools_json, instructions=None))]
    fn create_with_tools<'py>(
        &self,
        py: Python<'py>,
        model: String,
        input: String,
        tools_json: String,
        instructions: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let tools_val: Vec<serde_json::Value> = serde_json::from_str(&tools_json)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

            let tools: Vec<ResponseTool> = tools_val
                .into_iter()
                .map(|t| ResponseTool::Function {
                    name: t["name"].as_str().unwrap_or("").to_string(),
                    description: t["description"].as_str().map(String::from),
                    parameters: t.get("parameters").cloned(),
                    strict: None,
                })
                .collect();

            let mut req = ResponseCreateRequest::new(&model)
                .input(input.as_str())
                .tools(tools);
            if let Some(inst) = instructions {
                req = req.instructions(inst);
            }

            let resp = client.responses().create(req).await.map_err(to_py_err)?;
            response_to_dict(&resp)
        })
    }

    /// Raw request — send any JSON, get raw JSON back.
    #[pyo3(signature = (request_json,))]
    fn create_raw<'py>(
        &self,
        py: Python<'py>,
        request_json: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let body: serde_json::Value = serde_json::from_str(&request_json)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            let resp: serde_json::Value = client
                .responses()
                .create_raw(&body)
                .await
                .map_err(to_py_err)?;
            Ok(resp.to_string())
        })
    }

    /// Create streaming response — returns an async iterator of JSON strings
    #[pyo3(signature = (model, input, max_output_tokens=None))]
    fn create_stream<'py>(
        &self,
        py: Python<'py>,
        model: String,
        input: String,
        max_output_tokens: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let mut req = ResponseCreateRequest::new(&model).input(input.as_str());
            if let Some(max) = max_output_tokens {
                req = req.max_output_tokens(max);
            }
                
            use futures_util::StreamExt;
            let mut stream = client.responses().create_stream(req).await.map_err(to_py_err)?;
            
            let (tx, rx) = tokio::sync::mpsc::channel(32);
            
            tokio::spawn(async move {
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(event) => {
                            let s = serde_json::to_string(&event).unwrap_or_default();
                            if tx.send(Ok(s)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e.to_string())).await;
                            break;
                        }
                    }
                }
            });

            Python::with_gil(|py| {
                Py::new(py, PyResponseStream {
                    receiver: std::sync::Arc::new(tokio::sync::Mutex::new(rx)),
                }).map(|p| p.into_any())
            })
        })
    }
}

/// Convert Response to a Python-friendly dict string (JSON).
fn response_to_dict(resp: &::openai_oxide::types::responses::Response) -> PyResult<String> {
    let fcs: Vec<serde_json::Value> = resp
        .function_calls()
        .into_iter()
        .map(|fc| {
            serde_json::json!({
                "name": fc.name,
                "arguments": fc.arguments,
                "call_id": fc.call_id,
            })
        })
        .collect();

    let usage = resp.usage.as_ref().map(|u| {
        serde_json::json!({
            "input_tokens": u.input_tokens,
            "output_tokens": u.output_tokens,
            "total_tokens": u.total_tokens,
        })
    });

    let result = serde_json::json!({
        "id": resp.id,
        "model": resp.model,
        "text": resp.output_text(),
        "status": resp.status,
        "function_calls": fcs,
        "usage": usage,
    });

    Ok(result.to_string())
}

/// Python module.
#[pymodule]
fn openai_oxide(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<PyResponseStream>()?;
    Ok(())
}
