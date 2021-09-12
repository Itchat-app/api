import * as web from 'express-decorators'
import { Response, Request, NextFunction } from '@tinyhttp/app'
import { HTTPError } from '../../errors'
import { Invite } from '../../structures'
import { Permissions } from '../../utils'

@web.basePath('/servers/:server_id/invites')
export class ServerInviteController {
	@web.use()
	async hasAccess(req: Request, _res: Response, next: NextFunction): Promise<void> {
		const server = req.user.servers.getItems().find((s) => {
			return s._id === req.params.server_id
		})

		if (!server) {
			throw new HTTPError('UNKNOWN_SERVER')
		}

		Object.defineProperty(req, 'server', {
			value: server
		})

		next()
	}

	@web.get('/')
	async fetchMany(req: Request, res: Response): Promise<void> {
		const limit = 100 // TODO: Add Limit option

		const invites = await Invite.find({
			channel: {
				server: {
					_id: req.server._id
				}
			}
		}, { limit })

		res.json(invites)
	}

	@web.get('/:invite_code')
	async fetchOne(req: Request, res: Response): Promise<void> {
		const invite = await Invite.find({
			code: req.params.invite_code,
			channel: {
				server: {
					_id: req.server._id
				}
			}
		})

		if (!invite) {
			throw new HTTPError('UNKNON_INVITE')
		}

		res.json(invite)
	}

	@web.post('/:channel_id')
	async create(req: Request, res: Response): Promise<void> {
		const channel = await req.server.channels.matching({
			where: {
				_id: req.params.channel_id
			}
		}).then(([_]) => _)

		if (!channel) {
			throw new HTTPError('UNKNOWN_CHANNEL')
		}

		const permissions = await Permissions.fetch(req.user, req.server, channel)

		if (!permissions.has(Permissions.FLAGS.INVITE_OTHERS)) {
			throw new HTTPError('MISSING_PERMISSIONS')
		}

		const invite = await Invite.from({
			inviter: req.user,
			channel
		}).save()

		res.json({ code: invite.code })
	}
}