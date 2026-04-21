import { FormValidation } from '@/types'
import { Context } from 'hono'
import { ContentfulStatusCode } from 'hono/utils/http-status'

export const apiResponse = (c: Context, data?: any) => {
  if (data == undefined) {
    data = []
  }

  return c.json(data, 200)
}

export const apiStatusResponse = (
  c: Context,
  data?: any,
  message?: string,
  errors?: { [key: string]: string },
  code?: number
) => {
  if (data == undefined) {
    data = []
  }

  if (message == undefined) {
    message = 'OK'
  }

  if (code == undefined) {
    code = 200
  }

  const ret = {
    status: code,
    message: message,
    data: data
  } as FormValidation

  if (errors) {
    ret.errors = errors
  }

  return c.json(ret, code as ContentfulStatusCode)
}

export const apiBadRequest = (
  c: Context,
  message: string,
  errors?: { [key: string]: string },
  code?: number
) => {
  if (code == undefined) {
    code = 400
  }

  if (errors == undefined) {
    errors = {}
  }

  return apiStatusResponse(c, [], message, errors, code)
}

export const apiServerError = (c: Context) => {
  return c.json({
    message: 'API server error'
  }, 500)
}
