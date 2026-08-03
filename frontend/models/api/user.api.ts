import type { IUser } from "../entities/user.entities"
import type { IMetaPagination } from "./meta.api";
import type { TStatus } from "./status.api"

export interface IUserUpdate {
  data: IUser
  status: TStatus
}

export interface IUsersResponse {
  data: IUser[];
  status: TStatus;
  meta: IMetaPagination;
}
