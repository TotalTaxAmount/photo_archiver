<script>
  import GoogleLogin from "$lib/GoogleLogin.svelte";
  import Logout from "$lib/Logout.svelte";

</script>

<div class="container">
  <span class="google-login-button button"><GoogleLogin /></span>
  <span class="logout-button button"><Logout /></span>
</div>

<style>
  .container {
    display: flex;
    align-content: center;
  }

  .logout-button {
    position: absolute;
    top: 0;    
    right: 0; 
  }

  .button {
    padding: .2rem;
  }
</style>