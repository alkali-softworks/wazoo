class ApiConfig {
  port = 0
  baseUrl = ''

  updateConfig(port: number) {
    this.port = port
    this.baseUrl = `http://localhost:${port}`
  }
}

export const API_CONFIG = new ApiConfig()
