import { Entity, Property, wrap, FilterQuery, FindOptions } from 'mikro-orm'
import { ChannelTypes, Channel } from './Channel'
import db from '../database'

export interface CreateGroupOptions extends Omit<Partial<Group>, 'type'> {
    name: string
    ownerId: string
    recipients: string[]
}

@Entity({ tableName: 'channels' })
export class Group extends Channel {
    @Property()
    readonly type = ChannelTypes.GROUP

    @Property()
    name!: string

    @Property({ nullable: true })
    description?: string

    @Property({ nullable: true })
    icon?: string

    @Property()
    ownerId!: string

    @Property()
    recipients: string[] = []

    static from(options: CreateGroupOptions): Group {
        return wrap(new Group().setID()).assign(options)
    }

    static find(query: FilterQuery<Group>, options?: FindOptions<Group>): Promise<Group[]> {
        return db.get(Group).find(query, options)
    }

    static findOne(query: FilterQuery<Group>): Promise<Group | null> {
        return db.get(Group).findOne(query)
    }

    static count(query: FilterQuery<Group>): Promise<number> {
        return db.get(Group).count(query)
    }

    async save(options?: Partial<Group>): Promise<this> {
        await db.get(Group).persistAndFlush(options ? wrap(this).assign(options) : this)
        return this
    }
}