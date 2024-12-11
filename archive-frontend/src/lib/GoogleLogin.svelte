<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import googleIcon from '$lib/assets/google.svg';
  import axios from 'axios';
  import { authToken } from './stores/auth';

  let googleUser = false;
  let userPfpUrl = '';
  let username = '';

  const fetchUserInfo = async () => {
    try {
      const response = await axios.get('/api/users/userinfo', {
        headers: { "Authorization": `Bearer ${$authToken}` }
      });

      if (response.status === 200 && response.data.google) {
        googleUser = true;
        userPfpUrl = response.data.google.pfp_url;
        username = response.data.google.username;
      } else {
        googleUser = false;
      }
    } catch (error) {
      console.error('Error fetching user info:', error);
      googleUser = false;
    }
  };

  const login = async () => {
    try {
      const oauthUrl = await axios.get('/api/users/oauth/url', {
        headers: { "Authorization": `Bearer ${$authToken}` }
      });

      if (oauthUrl.status === 200) {
        window.location = oauthUrl.data.oauth_url;
      }
    } catch (error) {
      console.error('Error during login:', error);
    }
  };

  onMount(() => {
    fetchUserInfo();
  });
</script>
<!-- {#if googleUser}
{:else}
{/if} -->

<div class="container">
    <button type="submit" on:click={login} class="login-button">
      {#if googleUser}
        <img src={userPfpUrl} class="icon" width="24px" alt="G" />
      {:else}
        <img src={googleIcon} class="icon" width="24px" alt="G" /> 
      {/if}
      <span class="text">
        {#if googleUser === undefined}
          <span>Loading...</span>
        {:else if googleUser}
          Signed in as {username}
        {:else}
          Login to Google
        {/if}
      </span>
    </button>
</div>

<style>
.login-button {
  display: flex;
  align-items: center; 
  padding: 10px;
  font-size: 16px;
  background-color: #ffffff;
  color: rgb(0, 0, 0);
  border: none;
  cursor: pointer;
  border: 2px solid black;
  border-radius: 3px;
  margin: 10px;
}

.login-button .icon {
  width: 20px;
  height: 20px;
  margin-right: 8px;
  border-radius: 100%;
}

.login-button:hover {
  background-color: #dbdbdb; 
}
</style>
