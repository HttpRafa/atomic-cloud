---
--- Created by rafael.
--- DateTime: 23.05.24 00:43
---
local api = require("drivers.pelican.api")

function Init()
    print("Starting pelican driver for controller v" .. controller.version)
    api.get_servers()
end

function StopServer(server)
end

function StartServer(server)
end