-- lazycmd plugin configuration
lc.config {
  plugins = {
    { dir = 'plugins/align-test.lazycmd' },
    { dir = 'plugins/highlight-test.lazycmd' },
    { dir = 'plugins/interactive-test.lazycmd' },
    { dir = 'plugins/select-test.lazycmd' },
    { dir = 'plugins/quick-access-tools.lazycmd' },
    -- Local directory plugin example:
    -- { dir = 'plugins/myplugin.lazycmd' },
    'urie96/process.lazycmd',
    'urie96/himalaya.lazycmd',
    'urie96/systemd.lazycmd',
    'urie96/launchd.lazycmd',
    {
      'urie96/docker.lazycmd',
      dependencies = { 'urie96/promise.lazycmd' },
    },
    {
      'urie96/memos.lazycmd',
      config = function()
        require('memos').setup {
          token = os.getenv 'MEMOS_TOKEN',
          base_url = 'https://memos.lubui.com:8443',
        }
      end,
    },
  },
}
