import { log } from '@/lib/utils'

class Api {
  public async get<T>(endpoint: string): Promise<T> {
    const url = this.getUrl(endpoint)
    return fetch(url).then((r) => r.json())
  }

  public async post<T>(endpoint: string, data: object | []): Promise<T> {
    return this.call(endpoint, data, 'POST')
  }

  public async put<T>(endpoint: string, data: object | []): Promise<T> {
    return this.call(endpoint, data, 'PUT')
  }

  public async delete<T>(endpoint: string, data: object | []): Promise<T> {
    return this.call(endpoint, data, 'DELETE')
  }

  public async patch<T>(endpoint: string, data: object | []): Promise<T> {
    return this.call(endpoint, data, 'PATCH')
  }

  public async formPost<T>(endpoint: string, formData: FormData): Promise<T> {
    const url = this.getUrl(endpoint)
    return fetch(url, {
      method: 'POST',
      body: formData
    }).then((r) => r.json()) as Promise<T>
  }

  public baseUrl(): string {
    let port = localStorage.getItem('hono-port')
    if (!port || port === 'null') {
      port = '3000'
    }

    // Always want Hono server, not Vite dev server
    return `http://localhost:${port}`
  }

  private async call(endpoint: string, data: object | [], method: string) {
    const url = this.getUrl(endpoint)

    log('API CALL URL:', url)

    return fetch(url, {
      method: method,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      },
      body: JSON.stringify(data)
    }).then((r) => r.json())
  }

  private getUrl(endpoint: string): string {
    endpoint = endpoint.startsWith('/') ? endpoint : `/${endpoint}`
    const url = this.baseUrl()
    return `${url}/api${endpoint}`
  }
}

export default new Api()
