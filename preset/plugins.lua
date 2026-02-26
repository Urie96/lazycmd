return {
  {
    'memos',
    config = function()
      require('memos').setup {
        token = 'eyJhbGciOiJIUzI1NiIsImtpZCI6InYxIiwidHlwIjoiSldUIn0.eyJuYW1lIjoidXJpZSIsImlzcyI6Im1lbW9zIiwic3ViIjoiMSIsImF1ZCI6WyJ1c2VyLmFjY2Vzcy10b2tlbiJdLCJpYXQiOjE3NzA3MDg3MTN9.kOSggRwp4DeWIRRaZU9V9lb480BXsAL8limZ0nYxS3w',
        base_url = 'https://memos.lubui.com:8443',
      }
    end,
  },
  {
    'process',
    config = function() require('process').setup() end,
  },
}
