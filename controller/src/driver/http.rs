use mlua::{Lua, Table, LuaSerdeExt};

enum Parser {
    None,
    Json,
}

pub fn set_http_functions(lua: &Lua) {
    let globals = lua.globals();

    let table = lua.create_table().unwrap();
    {
        table.set("get", lua.create_async_function(get).unwrap()).unwrap();
        table.set("get_json", lua.create_async_function(get_json).unwrap()).unwrap();
    }

    globals.set("http", table).unwrap();
}

async fn get(lua: &Lua, uri: String) -> Result<Table, mlua::Error> {
    let table = lua.create_table().unwrap();
    let response = minreq::get(uri).send();

    build_table(response, lua, &table, Parser::None);
    Ok(table)
}

async fn get_json(lua: &Lua, uri: String) -> Result<Table, mlua::Error> {
    let table = lua.create_table().unwrap();
    let response = minreq::get(uri).send();

    build_table(response, lua, &table, Parser::Json);
    Ok(table)
}

fn build_table(response: Result<minreq::Response, minreq::Error>, lua: &Lua, table: &Table, parser: Parser) {
    match response {
        Ok(response) => {
            table.set("success", true).unwrap();
            table.set("code", response.status_code).unwrap();

            match response.as_str() {
                Ok(body) => match parser {
                    Parser::Json => {
                        match serde_json::from_str::<serde_json::Value>(body) {
                            Ok(json_body) => {
                                let lua_value = lua.to_value(&json_body).unwrap();
                                table.set("body", lua_value.as_table().unwrap()).unwrap();
                            },
                            Err(err) => {
                                eprintln!("Failed to parse body of http request as json: {}", err);
                            }
                        }
                    }
                    _ => {
                        table.set("body", body).unwrap();
                    }
                },
                Err(err) => {
                    eprintln!("Failed to read body of http response: {}", err);
                }
            }
        }
        Err(_) => {
            table.set("success", false).unwrap();
        }
    }
}