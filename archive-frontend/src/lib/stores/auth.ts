import { writable } from "svelte/store";
import axios from "axios";
import { createCookieStorage, persist, type PersistentStore } from "@macfja/svelte-persistent-store";

let tokenStorage; // You can customize options here
if (typeof window !== 'undefined') {
  tokenStorage = createCookieStorage();
}
export const authToken = tokenStorage ?  persist(writable(null), tokenStorage, 'session-token') : writable(null);


export const validateToken = async (token: string) : Promise<boolean> => {
  try {
    const res = await axios.get('/api/users/validate', { headers: { "Authorization": `Bearer ${token}` } });
    if (res.status !== 200) {
      return false
    }

    return true;
  } catch (e: any) {
    console.error("Error validating token: ", e);
    return false;
  }
}