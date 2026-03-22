import sys

with open('src/websocket.rs', 'r') as f:
    content = f.read()
content = content.replace("base.clone()", "base.to_string()")
with open('src/websocket.rs', 'w') as f:
    f.write(content)

with open('src/client.rs', 'r') as f:
    content = f.read()
content = content.replace("WsSession::connect(&self.config).await", "WsSession::connect(self.config.as_ref()).await")
with open('src/client.rs', 'w') as f:
    f.write(content)
