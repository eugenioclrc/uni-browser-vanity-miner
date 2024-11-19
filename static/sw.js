import init, { loop_hash } from './pkg/uni_vanity.js';

let ready = false;

// Initialize WASM and handle potential errors
init().then(() => {
    ready = true;
    self.postMessage({ status: "ready" });
}).catch((error) => {
    console.error("Failed to initialize WASM:", error);
    self.postMessage({ status: "init_failed", error: error.message });
});

self.onmessage = (e) => {
    if (!ready) {
        self.postMessage({ status: "not_ready" });
        return;
    }

    const { wallet, bestscore, times } = e.data;

    try {
        // Compute results using loop_hash
        const _results = loop_hash(wallet, times, bestscore);

        // Construct results object using correct fields
        const results = {
            address: _results.address || null,
            score: _results.score || null,
            salt: _results.salt || null,
        };

        self.postMessage({ status: "loop", results });
    } catch (error) {
        // Handle any errors during loop_hash
        console.error("Error in loop_hash:", error);
        self.postMessage({ status: "error", error: error.message });
    }
};
