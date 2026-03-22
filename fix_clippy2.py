with open('src/rate_limit.rs', 'r') as f:
    content = f.read()

content = content.replace("pub struct RateLimitTracker", "#[derive(Default)]\npub struct RateLimitTracker")
content = content.replace('''impl Default for RateLimitTracker {
    fn default() -> Self {
        Self {
            info: RateLimitInfo::default(),
        }
    }
}''', "")

with open('src/rate_limit.rs', 'w') as f:
    f.write(content)
