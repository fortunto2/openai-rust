const { Client } = require('../index.js');

async function main() {
    console.log("Testing Node.js WebSockets powered by openai-oxide (Rust)...");
    const client = new Client();
    
    console.log("Connecting WS...");
    const session = await client.wsSession();
    
    console.time("WS Request 1");
    const res1 = await session.send("gpt-4o-mini", "Say ping");
    console.timeEnd("WS Request 1");
    console.log("->", res1.output[0].content[0].text);

    console.time("WS Request 2 (Hot connection)");
    const res2 = await session.send("gpt-4o-mini", "Say pong");
    console.timeEnd("WS Request 2 (Hot connection)");
    console.log("->", res2.output[0].content[0].text);

    await session.close();
    console.log("Done.");
}

main();
