import { writable } from "svelte/store";
import axios from "axios";
import Cookies from "js-cookie";
import { navigate } from "svelte-routing";

export const isAuthenticated = writable(false);

export const validateToken = async () => {
  const token = Cookies.get("session-token");
  if (!token) {
    console.log("no token")
    isAuthenticated.set(false);
    navigate('/login');
  }

  try {
    const res = await axios.post("http://localhost:5173/api/users/validate", { token: token });
    if (res.status === 200) {
      isAuthenticated.set(true)
    } else {
      Cookies.remove("session-token")
      navigate('/login');
    }
  } catch (e: any) {
    isAuthenticated.set(false);
    console.error(e);
    navigate('/login');
  } 
}