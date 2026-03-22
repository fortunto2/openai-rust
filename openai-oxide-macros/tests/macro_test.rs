use openai_oxide_macros::openai_tool;

#[openai_tool(description = "Get the current weather in a given location")]
fn get_weather(location: String, unit: Option<String>) -> String {
    format!("Weather in {location} is 72 {unit:?}")
}

#[test]
fn test_macro_generation() {
    let tool_def = get_weather_tool();
    assert_eq!(tool_def["type"], "function");
    assert_eq!(tool_def["function"]["name"], "get_weather");
    assert_eq!(
        tool_def["function"]["description"],
        "Get the current weather in a given location"
    );

    let props = &tool_def["function"]["parameters"]["properties"];
    assert_eq!(props["location"]["type"], "string");
    assert_eq!(props["unit"]["type"], "string");

    let required = tool_def["function"]["parameters"]["required"]
        .as_array()
        .unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "location");
}
