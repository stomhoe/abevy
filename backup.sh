#!/bin/bash
echo "[$(date)] Backup started" >> /home/stefan/abevy/backup.log

repo_dir="/home/stefan/abevy"
worktree_dir="/home/stefan/abevy/.backup-worktree"

cd "$repo_dir" || exit 1

git fetch origin backups master 2>>/home/stefan/abevy/backup.log

if [ ! -d "$worktree_dir/.git" ]; then
	git worktree add -B backups "$worktree_dir" origin/backups 2>>/home/stefan/abevy/backup.log
fi

git -C "$worktree_dir" checkout backups 2>>/home/stefan/abevy/backup.log
git -C "$worktree_dir" reset --hard origin/backups 2>>/home/stefan/abevy/backup.log
git -C "$worktree_dir" merge master -m "Hourly backup $(date +%Y-%m-%d\ %H:%M:%S)" --no-edit 2>>/home/stefan/abevy/backup.log
git -C "$worktree_dir" push origin backups 2>>/home/stefan/abevy/backup.log || echo "[$(date)] PUSH FAILED" >> /home/stefan/abevy/backup.log

echo "[$(date)] Backup completed" >> /home/stefan/abevy/backup.log
