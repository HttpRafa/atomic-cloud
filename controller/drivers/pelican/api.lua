---
--- Created by rafael.
--- DateTime: 23.05.24 00:43
---
local module = {}
module.endpoint = "https://api.papermc.io/v2/projects/folia/versions/1.20.4/builds"

function module.get_servers()
    print(http.get_json(module.endpoint).body.project_id)
end

return module