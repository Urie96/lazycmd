lc.config {
  plugins = {
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
  },
}
