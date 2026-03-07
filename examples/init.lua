lc.config {
  plugins = {
    {
      'highlight-test',
      config = function() require('highlight-test').setup() end,
    },
    {
      'interactive-test',
      config = function() require('interactive-test').setup() end,
    },
    {
      'select-test',
      config = function() require('select-test').setup() end,
    },
    {
      'memos',
      config = function()
        require('memos').setup {
          token = os.getenv 'MEMOS_TOKEN',
          base_url = 'https://memos.lubui.com:8443',
        }
      end,
    },
    {
      'process',
      config = function() require('process').setup() end,
    },
    {
      'himalaya',
      config = function() require('himalaya').setup() end,
    },
    {
      'systemd',
      config = function() require('systemd').setup() end,
    },
  },
}
