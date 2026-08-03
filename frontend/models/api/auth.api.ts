import type { IUser } from "../entities/user.entities"
import type { TStatus } from "./status.api"

export interface IAuthMeValue {
  data: IUser
  status: TStatus
}