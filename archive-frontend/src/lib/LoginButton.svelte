<script lang="ts">
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
        console.error("No oauth url");
      }
    } catch (e) {
      console.error('Failed to fetch oauth url: ', e)
    }
  }
</script>

<button onclick={() => login()}>
  Login to google
</button>
