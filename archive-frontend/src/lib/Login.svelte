<script lang="ts">
  import type { Engine } from '@tsparticles/engine';
  import axios from 'axios';
  import Cookies from 'js-cookie';
  import { jwtDecode } from 'jwt-decode';
  import Particles, { particlesInit } from '@tsparticles/svelte';
  import { loadSlim } from '@tsparticles/slim';
  import { writable } from 'svelte/store';
  import { onMount } from 'svelte';

  let username = '';
  let password = '';
  let confirmPassword = '';
  let message = '';
  let isRegistering = false;

  interface JwtPayload {
    exp: number;
    id: number;
  }

  const toggleForm = () => {
    isRegistering = !isRegistering;
    message = '';
    password = '';
    confirmPassword = '';
  };

  const register = async () => {
    if (password !== confirmPassword) {
      message = "Passwords do not match!";
      return;
    }
    try {
      const response = await axios.post('/api/users/new', { username, password });
      if (response.status === 200) {
        message = 'Registration successful!';
        toggleForm();
      }
    } catch (error: any) {
      message = `${error.response?.data?.error || 'Registration failed'}`;
    }
  };

  const login = async () => {
    try {
      const response = await axios.post('/api/users/login', { username, password });
      if (response.status === 200 && response.data.token) {
        const token = response.data.token;

        const decoded: JwtPayload = jwtDecode(token);
        const exp = new Date(decoded.exp * 1000);

        Cookies.set('session-token', token, {
          expires: exp,
        });
        window.location.href = "/";
      }
    } catch (error: any) {
      message = `${error.response?.data?.error || 'Login failed'}`;
    }
  };

  let particleOptions = {
    particles: {
      // background: {
      //   color: { value: '#fff' }
      // },
      color: { value: '#fff' },
      number: { value: 100 },
      size: { value: 3 },
      move: {
        enable: true,
        speed: 2,
      },
      links: {
        enable: true,
        color: '#fff',
        distance: 200,
      },
      shape: { type: 'circle' },
    },
  };

  let width = writable(0);
  let height = writable(0);

  $: {
    const particleDensity = 0.000047;
    const totalParticles = Math.round($width * $height * particleDensity);

    particleOptions.particles.number.value = totalParticles;
  }

  onMount(() => {
    const updateDimensions = () => {
      width.set(window.innerWidth);
      height.set(window.innerHeight);
    };

    updateDimensions();
    window.addEventListener('resize', updateDimensions);
    return () => window.removeEventListener('resize', updateDimensions);
  })

  void particlesInit(async (engine: Engine) => {
    await loadSlim(engine);
  });
</script>

<style>
  .particles-container {
    background-color: var(--background-color);
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: -1; 
  }

  .login-container {
    background: var(--background-color);
    display: flex;
    flex-direction: column;
    width: 400px;
    margin: auto;
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: 5px;
    box-shadow: 0px 10px 30px rgba(0, 0, 0, 0.4);
  }

  h1 {
    font-size: 2rem;
    margin: 20px auto 40px;
  }

  input {
    margin-bottom: 1rem;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 5px;
  }

  button {
    padding: 0.5rem;
    margin: 0.5rem;
    border: none;
    border-radius: 5px;
    background: #007bff;
    color: var(--foreground-color);
    cursor: pointer;
  }

  button:hover {
    background: #0056b3;
  }

  .message {
    margin-top: 1rem;
    color: #d9534f;
    text-align: center;
  }

  .toggle-link {
    margin-top: 1rem;
    color: #007bff;
    cursor: pointer;
    text-align: center;
  }
  .toggle-link:hover {
    text-decoration: underline;
  }
</style>

<div class="particles-container">
  <Particles id="particles" class="" options="{particleOptions}" />
</div>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="login-container">
  <h1>Photo Archiver</h1>
  <input
    type="text"
    placeholder="Username"
    bind:value={username}
    required
  />
  <input
    type="password"
    placeholder="Password"
    bind:value={password}
    required
  />
  {#if isRegistering}
    <input
      type="password"
      placeholder="Confirm Password"
      bind:value={confirmPassword}
      required
    />
    <button on:click={register}>Register</button>
  {:else}
    <button on:click={login}>Login</button>
  {/if}
  <!-- svelte-ignore a11y_missing_attribute -->
  <a class="toggle-link" role="button" tabindex=0 on:click={toggleForm} >
    {isRegistering ? 'Already have an account? Login' : 'Don\'t have an account? Register'}
  </a>
  <div class="message">{message}</div>
</div>
