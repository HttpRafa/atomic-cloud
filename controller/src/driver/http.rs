use std::error::Error;
use log::error;
use mlua::{Lua, Table, LuaSerdeExt};

enum Parser {
    None,
    Json,
}

pub fn set_http_functions(lua: &Lua) {
    let globals = lua.globals();

    let table = lua.create_table().expect("Failed to create table");
    table.set("get", lua.create_async_function(get).expect("Failed to create async function"))
        .expect("Failed to set 'get' function");
    table.set("get_json", lua.create_async_function(get_json).expect("Failed to create async function"))
        .expect("Failed to set 'get_json' function");

    globals.set("http", table).expect("Failed to set globals");
}

async fn get(lua: &Lua, uri: String) -> Result<Table, mlua::Error> {
    let response = minreq::get(&uri).send();

    handle_table_build(response, &lua, &uri, Parser::None)
}

async fn get_json(lua: &Lua, uri: String) -> Result<Table, mlua::Error> {
    let response = minreq::get(&uri).send();

    handle_table_build(response, &lua, &uri, Parser::Json)
}

fn handle_table_build<'a>(response: Result<minreq::Response, minreq::Error>, lua: &'a Lua, uri: &String, parser: Parser) -> Result<Table<'a>, mlua::Error> {
    match response {
        Ok(response) => match build_table(response, lua, parser) {
            Ok(table) => Ok(table),
            Err(error) => {
                error!("Failed to convert http response from server({}) to lua table: {}", uri, error);
                Ok(build_simple_table(&lua, false).unwrap())
            }
        }
        Err(error) => {
            error!("Failed to read content from http server({}): {}", uri, error);
            return Ok(build_simple_table(&lua, false).unwrap());
        }
    }
}

fn build_simple_table(lua: &Lua, success: bool) -> Result<Table, Box<dyn Error>> {
    let table = lua.create_table()?;
    table.set("success", success)?;
    Ok(table)
}

fn build_table(response: minreq::Response, lua: &Lua, parser: Parser) -> Result<Table, Box<dyn Error>> {
    let table = build_simple_table(&lua, true)?;

    table.set("code", response.status_code)?;

    let body = response.as_str()?;

    if let Parser::Json = parser {
        let json_body: serde_json::Value = serde_json::from_str(body)?;
        let lua_value = lua.to_value(&json_body)?;
        if let Some(lua_table) = lua_value.as_table() {
            table.set("body", lua_table)?;
        } else {
            error!("Failed to convert lua value to table");
            table.set("body", lua.create_table()?)?;
        }
    } else {
        table.set("body", body)?;
    }

    Ok(table)
}