<script lang="ts">
  import googleIcon from '$lib/assets/google.svg'

  const login = async () => {
    console.log("hi")
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