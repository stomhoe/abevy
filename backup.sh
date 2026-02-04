#!/bin/bash
echo "[$(date)] Backup started" >> /home/stefan/abevy/backup.log
cd /home/stefan/abevy
git checkout backups 2>>/home/stefan/abevy/backup.log
git pull origin backups 2>>/home/stefan/abevy/backup.log
git merge master -m "Hourly backup $(date +%Y-%m-%d\ %H:%M:%S)" --no-edit 2>>/home/stefan/abevy/backup.log
git push origin backups 2>>/home/stefan/abevy/backup.log || echo "[$(date)] PUSH FAILED" >> /home/stefan/abevy/backup.log
git checkout master 2>>/home/stefan/abevy/backup.log
echo "[$(date)] Backup completed" >> /home/stefan/abevy/backup.log
