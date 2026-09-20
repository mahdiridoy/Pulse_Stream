"use client";

import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  ReactNode,
} from "react";
import { Media } from "@/types";

interface User {
  id: string;
  email: string;
  username: string;
  role: string;
}

interface AuthContextType {
  user: User | null;
  token: string | null;
  loading: boolean;
  login: (email: string, password: string) => Promise<{ error?: string }>;
  register: (
    email: string,
    username: string,
    password: string
  ) => Promise<{ error?: string }>;
  logout: () => void;
  isAuthenticated: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const saved = localStorage.getItem("pulse_token");
    if (saved) {
      setToken(saved);
      fetch("/api/v1/auth/me", {
        headers: { Authorization: `Bearer ${saved}` },
      })
        .then((r) => r.json())
        .then((data) => {
          if (data.success && data.data) {
            setUser(data.data);
          } else {
            localStorage.removeItem("pulse_token");
            setToken(null);
          }
        })
        .catch(() => {
          localStorage.removeItem("pulse_token");
          setToken(null);
        })
        .finally(() => setLoading(false));
    } else {
      setLoading(false);
    }
  }, []);

  const login = useCallback(
    async (email: string, password: string) => {
      try {
        const res = await fetch("/api/v1/auth/login", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ email, password }),
        });
        const data = await res.json();
        if (data.success && data.data) {
          localStorage.setItem("pulse_token", data.data.token);
          setToken(data.data.token);
          setUser(data.data.user);
          return {};
        }
        return { error: data.error?.message || "Login failed" };
      } catch {
        return { error: "Network error" };
      }
    },
    []
  );

  const register = useCallback(
    async (email: string, username: string, password: string) => {
      try {
        const res = await fetch("/api/v1/auth/register", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ email, username, password }),
        });
        const data = await res.json();
        if (data.success && data.data) {
          localStorage.setItem("pulse_token", data.data.token);
          setToken(data.data.token);
          setUser(data.data.user);
          return {};
        }
        return { error: data.error?.message || "Registration failed" };
      } catch {
        return { error: "Network error" };
      }
    },
    []
  );

  const logout = useCallback(() => {
    localStorage.removeItem("pulse_token");
    setToken(null);
    setUser(null);
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user,
        token,
        loading,
        login,
        register,
        logout,
        isAuthenticated: !!user,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
