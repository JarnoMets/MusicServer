export interface User {
  id: string;
  email: string;
  name: string;
  avatar_url?: string;
  is_admin: boolean;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface GoogleAuthRequest {
  code: string;
}
