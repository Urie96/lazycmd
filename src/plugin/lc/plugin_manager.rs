#[cfg(test)]
mod tests {
    use mlua::Lua;

    /// Inline Lua implementation of parse_plugin_spec and flatten_plugins for testing.
    /// Must be kept in sync with preset/lua/plugin_manager.lua
    const TEST_LUA: &str = r#"
local data_dir = os.getenv('HOME') .. '/.local/share/lazycmd/plugins'
local __lazycmd_config_base_dir = '/tmp/lazycmd-tests'

local function is_absolute_path(path)
  return path:match '^/' or path:match '^%a:[/\\]'
end

local function resolve_local_dir(dir)
  if type(dir) ~= 'string' or dir == '' then
    error 'plugin dir must be a non-empty string'
  end

  if dir:find '://' then
    error('plugin dir must be a relative or absolute path: ' .. dir)
  end

  if is_absolute_path(dir) then return dir end

  local base_dir = rawget(_G, '__lazycmd_config_base_dir') or '.'
  return base_dir .. '/' .. dir
end

local function plugin_name_from_dir(dir)
  local normalized = dir:gsub('[\\/]+$', '')
  local basename = normalized:match '([^/\\]+)$' or normalized
  return basename:match('^(.+)%.lazycmd$') or basename
end

local function parse_plugin_spec(spec)
  local source
  local dir

  if type(spec) == 'string' then
    source = spec
  elseif type(spec) == 'table' then
    source = spec[1]
    dir = spec.dir
  else
    return nil
  end

  if not source and not dir then return nil end

  local name
  if dir then
    name = source or plugin_name_from_dir(dir)
  elseif source:find('/') then
    local repo_name = source:match('^[^/]+/(.+)$')
    name = repo_name:match('^(.+)%.lazycmd$') or repo_name
  else
    name = source
  end

  local branch, tag, commit, config_fn, dependencies
  if type(spec) == 'table' then
    branch = spec.branch
    tag = spec.tag
    commit = spec.commit
    config_fn = spec.config
    dependencies = spec.dependencies
  end

  if not config_fn then
    config_fn = function()
      local ok, mod = pcall(require, name)
      if ok and mod and mod.setup then
        mod.setup()
      end
    end
  end

  local parsed_deps = {}
  if dependencies then
    for _, dep in ipairs(dependencies) do
      local dep_spec = parse_plugin_spec(dep)
      if dep_spec then
        parsed_deps[#parsed_deps + 1] = dep_spec
      end
    end
  end

  local result = {
    name = name,
    branch = branch,
    tag = tag,
    commit = commit,
    config = config_fn,
    dependencies = parsed_deps,
  }

  if dir then
    result.dir = resolve_local_dir(dir)
    result.is_remote = false
  elseif source:find('/') then
    result.repo = source
    result.url = 'https://github.com/' .. source .. '.git'
    result.install_path = data_dir .. '/' .. source:match('^[^/]+/(.+)$')
    result.is_remote = true
  else
    result.is_remote = false
  end

  return result
end

local function flatten_plugins(plugins)
  local seen = {}
  local result = {}

  local function add_plugin(spec)
    if not spec then return end
    if seen[spec.name] then return end
    seen[spec.name] = true

    for _, dep in ipairs(spec.dependencies or {}) do
      add_plugin(dep)
    end

    result[#result + 1] = spec
  end

  for _, p in ipairs(plugins or {}) do
    local spec = parse_plugin_spec(p)
    add_plugin(spec)
  end

  return result
end

local function get_remote_plugins(plugins)
  local result = {}
  for _, spec in ipairs(flatten_plugins(plugins or {})) do
    if spec and spec.is_remote then
      result[#result + 1] = spec
    end
  end
  return result
end

return { parse = parse_plugin_spec, flatten = flatten_plugins, remotes = get_remote_plugins }
"#;

    fn load_test_module(lua: &Lua) -> mlua::Result<(mlua::Function, mlua::Function, mlua::Function)> {
        let module: mlua::Table = lua.load(TEST_LUA).eval()?;
        let parse: mlua::Function = module.get("parse")?;
        let flatten: mlua::Function = module.get("flatten")?;
        let remotes: mlua::Function = module.get("remotes")?;
        Ok((parse, flatten, remotes))
    }

    #[test]
    fn test_parse_string_input() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        // Test: simple string input
        let result: mlua::Table = parse.call("owner/my-plugin.lazycmd")?;
        let name: String = result.get("name")?;
        assert_eq!(name, "my-plugin");

        Ok(())
    }

    #[test]
    fn test_parse_table_with_string() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        // Test: table with single string
        let spec = lua.create_table()?;
        spec.set(1, "owner/my-plugin.lazycmd")?;
        let result: mlua::Table = parse.call(spec)?;
        let name: String = result.get("name")?;
        assert_eq!(name, "my-plugin");

        Ok(())
    }

    #[test]
    fn test_parse_github_repo() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        spec.set(1, "owner/my-plugin.lazycmd")?;
        spec.set("branch", "main")?;
        let result: mlua::Table = parse.call(spec)?;

        let name: String = result.get("name")?;
        let repo: String = result.get("repo")?;
        let branch: String = result.get("branch")?;
        let is_remote: bool = result.get("is_remote")?;
        let url: String = result.get("url")?;

        assert_eq!(name, "my-plugin");
        assert_eq!(repo, "owner/my-plugin.lazycmd");
        assert_eq!(branch, "main");
        assert!(is_remote);
        assert!(url.contains("github.com"));

        Ok(())
    }

    #[test]
    fn test_parse_github_repo_without_suffix() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        spec.set(1, "owner/plain-repo")?;
        let result: mlua::Table = parse.call(spec)?;

        let name: String = result.get("name")?;
        let is_remote: bool = result.get("is_remote")?;
        assert_eq!(name, "plain-repo");
        assert!(is_remote);

        Ok(())
    }

    #[test]
    fn test_parse_local_plugin() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        spec.set(1, "process")?;
        let result: mlua::Table = parse.call(spec)?;

        let name: String = result.get("name")?;
        let is_remote: bool = result.get("is_remote")?;
        let url: mlua::Value = result.get("url")?;

        assert_eq!(name, "process");
        assert!(!is_remote);
        assert!(url.is_nil());

        Ok(())
    }

    #[test]
    fn test_parse_local_plugin_with_dir() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        spec.set("dir", "plugins/my-local.lazycmd")?;
        let result: mlua::Table = parse.call(spec)?;

        let name: String = result.get("name")?;
        let dir: String = result.get("dir")?;
        let is_remote: bool = result.get("is_remote")?;

        assert_eq!(name, "my-local");
        assert_eq!(dir, "/tmp/lazycmd-tests/plugins/my-local.lazycmd");
        assert!(!is_remote);

        Ok(())
    }

    #[test]
    fn test_parse_local_plugin_with_dir_and_name() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        spec.set(1, "myplugin")?;
        spec.set("dir", "/opt/plugins/custom.lazycmd")?;
        let result: mlua::Table = parse.call(spec)?;

        let name: String = result.get("name")?;
        let dir: String = result.get("dir")?;
        assert_eq!(name, "myplugin");
        assert_eq!(dir, "/opt/plugins/custom.lazycmd");

        Ok(())
    }

    #[test]
    fn test_parse_with_tag() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        spec.set(1, "owner/versioned.lazycmd")?;
        spec.set("tag", "1.0.0")?;
        let result: mlua::Table = parse.call(spec)?;

        let tag: String = result.get("tag")?;
        assert_eq!(tag, "1.0.0");

        Ok(())
    }

    #[test]
    fn test_parse_with_commit() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        spec.set(1, "owner/pinned.lazycmd")?;
        spec.set("commit", "abc1234567890def")?;
        let result: mlua::Table = parse.call(spec)?;

        let commit: String = result.get("commit")?;
        assert_eq!(commit, "abc1234567890def");

        Ok(())
    }

    #[test]
    fn test_parse_nil_source() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        let result: mlua::Value = parse.call(spec)?;
        assert!(result.is_nil());

        Ok(())
    }

    #[test]
    fn test_parse_with_dependencies() -> mlua::Result<()> {
        let lua = Lua::new();
        let (parse, _, _) = load_test_module(&lua)?;

        let spec = lua.create_table()?;
        spec.set(1, "owner/plugin.lazycmd")?;
        let deps = lua.create_table()?;
        deps.set(1, "owner/dep1.lazycmd")?;
        deps.set(2, "owner/dep2.lazycmd")?;
        spec.set("dependencies", deps)?;
        let result: mlua::Table = parse.call(spec)?;

        let deps: mlua::Table = result.get("dependencies")?;
        assert_eq!(deps.len()?, 2);

        let dep1: mlua::Table = deps.get(1)?;
        let dep1_name: String = dep1.get("name")?;
        assert_eq!(dep1_name, "dep1");

        Ok(())
    }

    #[test]
    fn test_flatten_plugins() -> mlua::Result<()> {
        let lua = Lua::new();
        let (_, flatten, _) = load_test_module(&lua)?;

        let plugins = lua.create_table()?;
        let dep = lua.create_table()?;
        dep.set(1, "owner/dep.lazycmd")?;
        let main_spec = lua.create_table()?;
        main_spec.set(1, "owner/main.lazycmd")?;
        main_spec.set("dependencies", dep)?;
        plugins.set(1, main_spec)?;
        plugins.set(2, "owner/other.lazycmd")?;

        let result: Vec<mlua::Table> = flatten.call(plugins)?;

        // Should have 3 plugins: dep (first due to dependency), main, other
        assert_eq!(result.len(), 3);
        // Dep should come before main
        let name0: String = result[0].get("name")?;
        let name1: String = result[1].get("name")?;
        assert_eq!(name0, "dep");
        assert_eq!(name1, "main");

        Ok(())
    }

    #[test]
    fn test_flatten_no_duplicates() -> mlua::Result<()> {
        let lua = Lua::new();
        let (_, flatten, _) = load_test_module(&lua)?;

        let plugins = lua.create_table()?;

        let p1 = lua.create_table()?;
        p1.set(1, "owner/p1.lazycmd")?;
        let dep1 = lua.create_table()?;
        dep1.set(1, "owner/shared.lazycmd")?;
        p1.set("dependencies", dep1)?;
        plugins.set(1, p1)?;

        let p2 = lua.create_table()?;
        p2.set(1, "owner/p2.lazycmd")?;
        let dep2 = lua.create_table()?;
        dep2.set(1, "owner/shared.lazycmd")?;
        p2.set("dependencies", dep2)?;
        plugins.set(2, p2)?;

        let result: Vec<mlua::Table> = flatten.call(plugins)?;

        // Should have 3 unique plugins, not 5
        assert_eq!(result.len(), 3);

        // Count how many times "shared" appears
        let mut shared_count = 0;
        for spec in &result {
            let name: String = spec.get("name")?;
            if name == "shared" {
                shared_count += 1;
            }
        }
        assert_eq!(shared_count, 1);

        Ok(())
    }

    #[test]
    fn test_get_remote_plugins_includes_dependencies() -> mlua::Result<()> {
        let lua = Lua::new();
        let (_, _, remotes) = load_test_module(&lua)?;

        let plugins = lua.create_table()?;
        let main = lua.create_table()?;
        main.set(1, "owner/main.lazycmd")?;

        let deps = lua.create_table()?;
        deps.set(1, "owner/dep.lazycmd")?;
        deps.set(2, "local-helper")?;
        main.set("dependencies", deps)?;

        plugins.set(1, main)?;

        let result: Vec<mlua::Table> = remotes.call(plugins)?;
        assert_eq!(result.len(), 2);

        let dep_name: String = result[0].get("name")?;
        let main_name: String = result[1].get("name")?;
        assert_eq!(dep_name, "dep");
        assert_eq!(main_name, "main");

        Ok(())
    }
}
