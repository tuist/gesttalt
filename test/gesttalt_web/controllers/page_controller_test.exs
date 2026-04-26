defmodule GesttaltWeb.PageControllerTest do
  use GesttaltWeb.ConnCase

  test "GET /", %{conn: conn} do
    conn = get(conn, ~p"/")

    assert html_response(conn, 200) =~
             "Federated publications, started with a clean Phoenix baseline"
  end
end
