const namedQueue = require('named-queue')
const consts = require('./consts')
const addons = require('../lib/addons')
const mapMetaToNotifs = require('./lib/mapMetaToNotifs')

const queue = new namedQueue(feedsWorker, consts.FEED_QUEUE_CONCURRENCY)

function feedsWorker(task, cb) {
	const redis = task.redis
	const feed = task.feed

	const isIntro = feed.id == consts.INTRO_FEED_ID
	const isSeries = feed.id.startsWith('tt')
	const isChannel = feed.id.startsWith('yt_id')

	if (isIntro) {
		// @TODO: remove this; we should do intro notifications in a client-side way
		const cmd = redis.multi()
		cmd.hset(consts.FEEDS_UPDATED_KEY, feed.id, Date.now())
		cmd.zadd(consts.FEEDS_MTIME_KEY+feed.id, [consts.INTRO_GUIDE_PUBLISHED, consts.INTRO_NOTIF._id])
		cmd.set(consts.NOTIFS_KEY+consts.INTRO_NOTIF._id, JSON.stringify(consts.INTRO_NOTIF))
		cmd.exec(cb)
		return
	}

	if (!(isSeries || isChannel)) {
		cb(new Error('unsupported feed ID: '+task.id))
		return
	}

	const addon = isSeries ? addons.cinemeta : addons.channels
	const type = isSeries ? 'series' : 'channel'
	const lastVideos = isSeries ? consts.SERIES_LASTVIDEOS : consts.CHANNEL_LASTVIDEOS
	const cacheBreak = Math.floor(Date.now() / consts.CACHE_BREAK_FREQ)

	addon.get('meta', type, feed.id, { lastVideos, cacheBreak })
	.then(function(resp) {
		const newerThan = Date.now() - consts.MAX_NOTIF_AGE
		const notifs = mapMetaToNotifs(resp.meta)
			.filter(x => x.published.getTime() > newerThan)

		const cmd = redis.multi()
		const toAdd = notifs.map(n => [n.published.getTime(), n._id])

		// Update the notification mtimes for that feed
		const mtimesKey = consts.FEEDS_MTIME_KEY+feed.id
		cmd.zremrangebyscore(mtimesKey, 0, newerThan)
		if (toAdd.length) {
			// flatten toAdd for zadd
			cmd.zadd(mtimesKey, toAdd.reduce((a, b) => a.concat(b), []))
		}

		// Save the notifications themselves
		notifs.forEach(function(notif) {
			// NOTE: this means the EXPIRES time will always be updated,
			// but this is OK because the notification will go out of the lastVideos window
			cmd.setex(consts.NOTIFS_KEY+notif._id, Math.floor(consts.MAX_NOTIF_AGE/1000), JSON.stringify(notif))
		})

		// Set the feed updated time
		// @TODO: NOTE: we can set the updated to now + a few months, if resp.meta.status === 'Ended' as an optimization; essentially 'snooze'
		cmd.hset(consts.FEEDS_UPDATED_KEY, feed.id, Date.now())

		cmd.exec(cb)
	})
	.catch(function(err) {
		cb(err)
	})
}

function updateFeeds(redis, feeds) {
	return Promise.all(feeds.map(function(feed) {
		return new Promise(function(resolve, reject) {
			queue.push({
				id: feed.id,
				feed: feed,
				redis: redis,
			}, function(err, res) {
				if (err) console.error('Error updating feed:', feed.id, err)
				resolve(res)
			})
		})
	}))
}

module.exports = updateFeeds