<div class="main-content">
  <slot></slot>
</div>

<style>
  :root {
    --background-color: #242424;
    --foreground-color: rgba(255, 255, 255, 0.87);

    font-family: Inter, system-ui, Avenir, Helvetica, sans-serif;
    color-scheme: light dark;

    font-synthesis: none;
  }

  .main-content {
    display: flex;
    flex-direction: column;
    place-items: center;
    height: 100vh;
  }

  @media (prefers-color-scheme: light) {
    :root {
      --foreground-color: #242424;
      --background-color: rgba(255, 255, 255, 0.87);
    }
  }
</style>