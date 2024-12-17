<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import axios from 'axios';
  import { authToken } from './stores/auth';

  

  const logout = async () => {
    try {
      axios.post('/api/users/logout', "", {
        headers: { "Authorization": `Bearer ${$authToken}` }
      }).then(res => {
        if (res.status === 200) {
       authToken.set(null);
       goto("/login")
      }
      });
    } catch (error) {
      console.error('Error during logout:', error);
    }
  };
</script>

<div class="container">
  <button type="submit" on:click={logout} class="logout-button">
    Logout
  </button>
</div>

<style>
.logout-button {
  display: flex;
  align-items: center; 
  padding: 10px;
  font-size: 16px;
  background-color: #e82121;
  color: rgb(255, 255, 255);
  border: none;
  cursor: pointer;
  border-radius: 3px;
  margin: 10px;
}


.logout-button:hover {
  background-color: #fb3232; 
}
</style>
