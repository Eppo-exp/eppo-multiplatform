# Tests that imports are available at old location (for
# backward-compatibility)


def test_config():
    from eppo_client.config import Config

    # Our docs import logger from eppo_client.config
    from eppo_client.config import AssignmentLogger


def test_assignment_logger():
    from eppo_client.assignment_logger import AssignmentLogger
