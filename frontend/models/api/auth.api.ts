import type { IUserResponse } from "../entities/user.entities"
import type { TStatus } from "./status.api"

export interface IAuthMeValue {
  data: IUserResponse
  status: TStatus
}