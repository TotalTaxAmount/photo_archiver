<script lang="ts">
  import type { Engine } from '@tsparticles/engine';
  import axios from 'axios';
  import Cookies from 'js-cookie'
  import { jwtDecode } from 'jwt-decode';
  import Particles, { particlesInit } from '@tsparticles/svelte';
  import { loadSlim } from '@tsparticles/slim';

  let username = '';
  let password = '';
  let message = '';

  interface JwtPayload {
    exp: number;
    username: string;
  }

  const register = async () => {
    try {
      const response = await axios.post('/api/users/new', { username, password });
      if (response.status === 200) {
        message = 'Registration successful!';
      }
    } catch (error: any) {
      message = `Error: ${error.response?.data?.error || 'Registration failed'}`;
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
        })
      }
    } catch (error: any) {
      message = `Error: ${error.response?.data?.error || 'Login failed'}`;
    }
  };

  const particleOptions = {
    particles: {
      color: { value: '#fff', },
      number: { value: 120 },
      size: { value: 3 },
      move: {
        enable: true,
        speed: 2,
      },
      links: {
        enable: true,
        color: '#fff',
        distance: 200
      },
      shape: { type: 'circle' }
    }
  }

  void particlesInit(async (engine: Engine) => {
    await loadSlim(engine);
  });
</script>

<style>
  html, body {
    height: 100%;
    margin: 0;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .particles-container {
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
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
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
    color: white;
    cursor: pointer;
  }
  button:hover {
    background: #0056b3;
  }
  .message {
    margin-top: 1rem;
    color: #d9534f;
  }
</style>

<div class="particles-container">
  <Particles
    id = "particles"
    class = ""
    options = "{particleOptions}"
  />
</div>

<div class="login-container">
  <h1>Photo Archiver login</h1>
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
  <button on:click={register}>Register</button>
  <button on:click={login}>Login</button>
  <div class="message">{message}</div>
</div>
