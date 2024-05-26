---
--- Created by rafael.
--- DateTime: 23.05.24 00:43
---
local api = require("drivers.pelican.api")

function Init()
    http.get_json("http://sdasdasdasdasdasdas")
    return {
        ["author"] = "HttpRafa",
        ["version"] = "0.1.0",
    }
end

function StopServer(server)
end

function StartServer(server)
end