var app = document.getElementById('app');
var h1 = document.createElement('h1');
h1.textContent = 'Client Rendered Heading';
app.appendChild(h1);
var p = document.createElement('p');
p.textContent = 'This paragraph and the heading above are injected entirely by ' +
  'client-side JavaScript after the page loads. The server response body ' +
  'contains only an empty app container, so any crawler relying on the raw ' +
  'server HTML will see almost no content here. A headless browser that ' +
  'executes JavaScript will instead observe the fully hydrated document with ' +
  'this substantial block of text rendered into the page for indexing.';
app.appendChild(p);
var p2 = document.createElement('p');
p2.textContent = 'This second paragraph adds more client-rendered content to ' +
  'ensure the word count is well above the minimum threshold for SSR detection. ' +
  'The additional words help verify that the crawler correctly identifies a ' +
  'significant gap between server-rendered and client-rendered content, making ' +
  'the SSR content missing flag more reliable and the SSR CSR diff percentage ' +
  'more pronounced in the test assertions.';
app.appendChild(p2);
