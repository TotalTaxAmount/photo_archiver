<script setup lang="ts">
  async function login() {
    try {
      let res = await fetch(`${window.location.origin}/api/oauth/new`, {
        method: 'GET'
      });

      if (!res.ok) {
        throw new Error(`Error: ${res.statusText}`);
      }

      let data = await res.json();

      if (data.oauth_url) {
        window.open(data.oauth_url, '_self');
      } else {
        console.error('No url')
      }
    } catch (e) {
      console.error('Failed to fetch oauth url: ', e);
    }
  }
</script>

<template>
  <!-- <div class="card">
    <button type="button" @click="count++">count is {{ count }}</button>
    <p>
      Edit
      <code>components/HelloWorld.vue</code> to test HMR
    </p>
  </div>

  <p>
    Check out
    <a href="https://vuejs.org/guide/quick-start.html#local" target="_blank"
      >create-vue</a
    >, the official Vue + Vite starter
  </p>
  <p>
    Learn more about IDE Support for Vue in the
    <a
      href="https://vuejs.org/guide/scaling-up/tooling.html#ide-support"
      target="_blank"
      >Vue Docs Scaling up Guide</a
    >.
  </p>
  <p class="read-the-docs">Click on the Vite and Vue logos to learn more</p> -->
  <div class="main">
    <button class="g-login" type="button" @click="login()">Login with google</button>
  </div>
</template>

<style scoped>
.read-the-docs {
  color: #888;
}
</style>
