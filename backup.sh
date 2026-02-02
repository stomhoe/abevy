#!/bin/bash
echo "[$(date)] Backup started" >> /home/stefan/abevy/backup.log
cd /home/stefan/abevy
git checkout backups 2>/dev/null
git pull origin backups 2>/dev/null
git merge master -m "Hourly backup $(date +%Y-%m-%d\ %H:%M:%S)" --no-edit 2>/dev/null
git push origin backups 2>/dev/null
git checkout master 2>/dev/null
echo "[$(date)] Backup completed" >> /home/stefan/abevy/backup.log
