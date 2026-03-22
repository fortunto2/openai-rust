const { Client } = require('../index.js');

async function main() {
    console.log("Testing Node.js bindings powered by openai-oxide (Rust)...");
    const client = new Client();
    
    const request = {
        model: "gpt-4o-mini",
        input: "Say hello to Node.js from Rust via NAPI!",
        instructions: "You are a helpful assistant.",
        temperature: 0.7
    };

    console.time("Request");
    console.log("Sending request:", request);
    try {
        const response = await client.createResponse(request);
        console.timeEnd("Request");
        
        console.log("\n--- Full Native JS Response Object ---");
        console.log(response);
        
        console.log("\n--- Assistant Message ---");
        console.log(response.output[0].content[0].text);
    } catch (e) {
        console.error("Error:", e);
    }
}

main();
