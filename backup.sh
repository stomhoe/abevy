#!/bin/bash
echo "[$(date)] Backup started" >> /home/stefan/abevy/backup.log

repo_dir="/home/stefan/abevy"

cd "$repo_dir" || exit 1

git fetch origin master 2>>/home/stefan/abevy/backup.log
git push --force origin master:backups 2>>/home/stefan/abevy/backup.log || echo "[$(date)] PUSH FAILED" >> /home/stefan/abevy/backup.log

echo "[$(date)] Backup completed" >> /home/stefan/abevy/backup.log
