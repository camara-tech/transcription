# User Requirements Document

## Feature: Transcription Script

As a user, I want a transcription script that won't transcribe previously successful transcriptions, so that I can avoid redundant work and save time.

### Scenario: Extract MP3 files from a directory
**Given** a directory containing MP3 files
**When** the script is executed
**Then** it should extract all MP3 files from the directory and its subdirectories

### Scenario: Identify already transcribed files
**Given** a destination directory containing previously transcribed files
**When** the script is executed
**Then** it should identify all transcribed files in the destination directory

### Scenario: Filter untranscribed files
**Given** a list of MP3 files and a list of transcribed files
**When** the script is executed
**Then** it should filter out the MP3 files that have already been transcribed

### Scenario: Transcribe MP3 files
**Given** a list of untranscribed MP3 files
**When** the script is executed
**Then** it should transcribe each MP3 file using `aTrain_core` and save the transcription

### Scenario: Copy transcription files to destination directory
**Given** a list of newly transcribed files and a destination directory
**When** the script is executed
**Then** it should copy the transcription files to the destination directory

### Scenario: Verbose mode
**Given** the script is executed with the `--verbose` flag
**When** the script is running
**Then** it should print detailed information about each step of the process
