import { Base, Role } from '.'
import { validator } from '../utils'
import sql from '../database'
import config from '../config'

export interface CreateMemberOptions extends Options<Member> {
  id: string
  server_id: string
}

export const UpdateMemberSchema = validator.compile({
  nickname: `string|min:0|max:${config.limits.member.nickname}|nullable`,
  roles: 'snowflake[]|unique|optional'
})


export class Member extends Base {
  nickname: Nullable<string> = null
  joined_at = Date.now()
  server_id!: string
  roles: string[] = []

  fetchRoles(): Promise<Role[]> {
    return Role.find({ id: this.roles })
  }

  static from(opts: CreateMemberOptions): Member {
    return Object.assign(new Member(), opts)
  }

  static async init(): Promise<void> {
    await sql.unsafe(`CREATE TABLE IF NOT EXISTS ${this.tableName} (
            id BIGINT PRIMARY KEY,
            joined_at TIMESTAMP NOT NULL,
            nickname VARCHAR(${config.limits.member.nickname}),
            server_id BIGINT NOT NULL,
            roles JSONB NOT NULL,
            FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
            FOREIGN KEY (id) REFERENCES users(id) ON DELETE CASCADE
        )`)
  }
}