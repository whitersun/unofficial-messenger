<template>
  <main>
    <section>
      <p>Messenger wrapper</p>
      <h1>Open Messenger in the desktop app</h1>
      <a href="https://www.messenger.com/">Open Messenger</a>
    </section>
  </main>
</template>

<style>
:root {
  color: #16181d;
  background: #f7f8fb;
  font-size: 16px;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-weight: 400;
  line-height: 1.5;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}
body {
  margin: 0;
}

main {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  box-sizing: border-box;
}

section {
  width: min(100%, 440px);
  text-align: center;
}

p {
  margin: 0 0 12px;
  color: #576071;
  font-size: 0.88rem;
  font-weight: 700;
  text-transform: uppercase;
}

h1 {
  margin: 0 0 24px;
  color: #12151c;
  font-size: 2rem;
  line-height: 1.15;
}

a {
  display: inline-flex;
  min-height: 44px;
  align-items: center;
  justify-content: center;
  padding: 0 18px;
  border-radius: 8px;
  color: #ffffff;
  background: #0a7cff;
  font-weight: 700;
  text-decoration: none;
  box-shadow: 0 10px 20px rgba(10, 124, 255, 0.24);
}
</style>
