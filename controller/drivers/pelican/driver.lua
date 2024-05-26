---
--- Created by rafael.
--- DateTime: 23.05.24 00:43
---
local inspect = require("drivers.libs.inspect")
local api = require("drivers.pelican.api")

function Init()
    return {
        ["author"] = "HttpRafa",
        ["version"] = "0.1.0",
    }
end

function InitNode(node)
    return true
end

function StopServer(server)
end

function StartServer(server)
end