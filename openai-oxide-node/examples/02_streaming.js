const { Client } = require('../index.js');

async function main() {
    console.log("Testing Node.js streaming powered by openai-oxide (Rust)...");
    const client = new Client();
    
    const request = {
        model: "gpt-4o-mini",
        input: "Write a haiku about Rust and Node.js",
    };

    console.log("Starting stream...\n");
    
    const stream = () => {
        let resolveNext;
        let rejectNext;
        const queue = [];
        let isDone = false;
        
        client.createStream(request, (err, event) => {
            if (err) {
                if (rejectNext) rejectNext(err);
                else queue.push({ err });
            } else if (event.type === 'done') {
                isDone = true;
                if (resolveNext) resolveNext({ done: true });
                else queue.push({ done: true });
            } else {
                if (resolveNext) {
                    resolveNext({ value: event, done: false });
                    resolveNext = null;
                } else {
                    queue.push({ value: event, done: false });
                }
            }
        });

        return {
            async next() {
                if (queue.length > 0) {
                    const nextItem = queue.shift();
                    if (nextItem.err) throw nextItem.err;
                    return nextItem;
                }
                if (isDone) return { done: true };
                
                return new Promise((resolve, reject) => {
                    resolveNext = resolve;
                    rejectNext = reject;
                });
            },
            [Symbol.asyncIterator]() { return this; }
        };
    };

    for await (const event of stream()) {
        if (event.type === 'response.output_text.delta') {
            process.stdout.write(event.delta);
        } else if (event.type === 'response.completed') {
            console.log("\n\n--- Stream Completed ---");
            console.log(`Tokens used: ${event.response.usage.total_tokens}`);
        }
    }
}

main();
