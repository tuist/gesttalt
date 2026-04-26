defmodule GesttaltWeb.PageController do
  use GesttaltWeb, :controller

  def home(conn, _params) do
    render(conn, :home)
  end
end
