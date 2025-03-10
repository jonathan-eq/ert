
class EE:
  """A class to replace the old python EE"""
  def __init__(self, address: str) -> None: ...
  
  def run(self) -> None:
    """Starts the EE"""
    
  def wait_for_finish(self) -> int:
    """Waits for the EE to wrap up
      :return: Returns the count of messages
    """