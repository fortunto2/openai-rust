import sys
with open('src/resources/responses.rs', 'r') as f:
    content = f.read()

target = '''            return Err(OpenAIError::HttpError {
                status: status_code,
                message: body,
            });'''

replacement = '''            return Err(OpenAIError::ApiError {
                status: status_code,
                message: body,
                type_: None,
                code: None,
            });'''

content = content.replace(target, replacement)
with open('src/resources/responses.rs', 'w') as f:
    f.write(content)
