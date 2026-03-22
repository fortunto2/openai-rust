import sys

with open('openai-oxide-node/src/lib.rs', 'r') as f:
    content = f.read()

target = 'match self.inner.post_stream_raw("/chat/completions", &body).await {'
replacement = 'match self.inner.chat().completions().create_stream_raw(&body).await {'
content = content.replace(target, replacement)

with open('openai-oxide-node/src/lib.rs', 'w') as f:
    f.write(content)
