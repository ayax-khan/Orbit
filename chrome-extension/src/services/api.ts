const API_BASE_URL = 'http://localhost:8080/api/v1';

/** Extract a useful error message from a failed API response. */
async function errorMessage(response: Response): Promise<string> {
  try {
    const body = await response.json();
    if (body && typeof body.error === 'string') {
      return body.error;
    }
  } catch {
    // fall through to status text
  }
  return response.statusText || `HTTP ${response.status}`;
}

export const api = {
  async post<T>(endpoint: string, data: unknown, token?: string): Promise<T> {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      throw new Error(await errorMessage(response));
    }

    return response.json() as Promise<T>;
  },

  async get<T>(endpoint: string, token?: string): Promise<T> {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      method: 'GET',
      headers: {
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
    });

    if (!response.ok) {
      throw new Error(await errorMessage(response));
    }

    return response.json() as Promise<T>;
  },

  async delete<T>(endpoint: string, token?: string): Promise<T> {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      method: 'DELETE',
      headers: {
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
    });

    if (!response.ok) {
      throw new Error(await errorMessage(response));
    }

    return response.json() as Promise<T>;
  },
};
