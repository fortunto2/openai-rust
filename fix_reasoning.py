import sys

with open('src/resources/chat/mod.rs', 'r') as f:
    content = f.read()

# I put it inside the struct but maybe inside the impl block incorrectly? Let's check where it went.
