export type CreatePresenceOptions = Partial<Presence>

export enum PresenceStatus {
	ONLINE,
	OFFLINE,
	IDLE,
	DND
}

export class Presence {
	status = PresenceStatus.OFFLINE
	ghost_mode = false
	static from(options: CreatePresenceOptions): Presence {
		const presence = new Presence()
		Object.assign(presence, options)
		return presence
	}
}