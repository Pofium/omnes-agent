import { fetchUser } from "./api";

export interface User {
  id: number;
}

export class Registry {
  add(u: User): void {
    fetchUser();
  }
}

export function bootstrap(): void {
  fetchUser();
}
