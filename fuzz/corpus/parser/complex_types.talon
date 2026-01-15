let dict = {"key": "value", "num": 42}
let nested = [{"a": 1}, {"b": 2}]

function process(data: map) -> list
    let result = []
    for key in data.keys()
        result.append(data[key])
    end
    return result
end
