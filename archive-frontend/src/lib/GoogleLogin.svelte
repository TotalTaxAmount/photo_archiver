<script lang="ts">
  import { goto } from '$app/navigation';
  import googleIcon from '$lib/assets/google.svg'
  import axios from 'axios';

  const login = async () => {
    let oauthUrl = await axios.get('/api/oauth/new');
    if (oauthUrl.status === 200) {
      window.location = oauthUrl.data.oauth_url;
    }
  }
</script>

<div class="container">
  <button type="button" on:click={login} class="login-button">
    <img src={googleIcon} class="login-icon" width="24px" alt="G"/> 
    <span class="login-text">
      Login to Google
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
}

.login-button .login-icon {
  width: 20px;
  height: 20px;
  margin-right: 8px;
}

.login-button:hover {
  background-color: #dbdbdb; 
}
</style>